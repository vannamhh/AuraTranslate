<!-- bmad:context -->
<!-- Verified 2026-08-25 against 69b19a8. Managed by bmad-project-context; edits inside this block are replaced on refresh. Keep anything you want preserved outside the markers. -->

## AuraTranslate

Không gian làm việc tra từ điển và dịch thuật, chạy hoàn toàn offline. Tauri v2 · Rust ở `src-tauri/` · Vue 3 + TypeScript ở `src/` · SQLite bundled. GPL-3.0-or-later. Quy hoạch ở `_bmad-output/planning-artifacts/`, story và sổ nợ ở `_bmad-output/implementation-artifacts/`.

## Policy

- Nhánh mặc định là `master`. Viết `branches: [main]` vào một workflow thì CI không bao giờ chạy và không lỗi nào được ném.
- Không commit tệp `.db`. Dòng `*.db` trong `.gitignore` là cố ý (AD-25) — dữ liệu từ điển đi qua GitHub Release + `dict-manifest.toml`.
- Trước khi thêm BẤT KỲ phụ thuộc nào (NFR15): mở tệp giấy phép trong nguồn ĐÃ TẢI mà đọc (`~/.cargo/registry/src/…`, `node_modules/…`), không tin nhãn registry; rồi ghi vào bảng Stack của spine TRƯỚC khi thêm. Chỉ giấy phép tương thích GPLv3 chiều đi vào.
- Đổi một bất biến kiến trúc là một `AD` mới trong spine, không phải một dòng mã.
- Cấp số `AD` mới thì quét CẢ spine LẪN mọi `ad-brief-*.md` chưa soạn thành AD — số lớn nhất trong spine không phải số kế tiếp. AD-48 va đúng vậy: hồ sơ hoàn tác 2026-08-17 giữ chỗ 48, lượt hộp thoại 2026-08-24 đo spine tới AD-47 rồi cấp lại 48, và B6 trong `sprint-status.yaml` vẫn mở với số đó.
- Hai phương án đều hợp lệ ⇒ nêu cả hai kèm số đo cho Ice chốt, đừng tự chọn rồi đi tiếp.
- Cây bẩn trước khi bắt đầu một story ⇒ commit riêng, trước, và hỏi Ice trước khi commit.

## Where things are

- Bất biến kiến trúc (AD-1…AD-48, bảng Stack, Consistency Conventions): `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md`
- Sổ nợ: `_bmad-output/implementation-artifacts/deferred-work.md`. Trạng thái story ở `sprint-status.yaml`; nội dung story ở chính tệp story.
- Luật theo thư mục, gắn theo vị trí: `src/AGENTS.md` · `src-tauri/AGENTS.md` · `scripts/AGENTS.md` · `tests/AGENTS.md` · `e2e/AGENTS.md` · `tools/dict-build/AGENTS.md`
- Lịch sử đo đạc và lý do đầy đủ: `_bmad-output/project-context.md` — tham chiếu sâu, không bắt buộc đọc.
- `docs/` là nguồn thô của `tools/dict-build`, không phải tài liệu cho agent — đừng đọc nó để hiểu dự án.

## Running and verifying

- `pre-push` chạy 11 cổng → vitest → build → `cargo test --locked`, trên **macOS của Ice**. Nó không nói gì về nửa Windows; CI chạy cả hai nền tảng mỗi lượt push, nên đọc lượt CI trước khi kết luận là xanh.
- Bộ e2e chạy ở NHỊP ĐÊM, không ở `push` (`schedule` + `workflow_dispatch`, chỉ macOS) — nên một lượt push xanh vẫn không nói gì về nó, và nửa Windows chưa từng chạy. Chạy tay: `npm run test:e2e`. Cách đọc một lượt đỏ: `e2e/AGENTS.md`.
- `check:scope` và `check:scope:bundled` ngoài `pre-push` có chủ ý: chúng dựng cửa sổ Tauri thật và cần cổng 1420 trống, nên trượt khi đang mở `npm run tauri dev`. CI có chạy; máy dev chạy tay.
- Thêm một cổng = sửa BA danh sách (`package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`), và `check:gates` canh cả ba cho mọi cổng `check:*`. `test:e2e` là ngoại lệ CÓ TÊN, chỉ được canh ở hai — xem `scripts/AGENTS.md`.
- Chạy `npm run build` TRƯỚC `cargo test`: thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch, không ở một assert.
- Không đánh dấu đạt bằng suy luận. Vế nào không nghiệm thu được ở tầng đang làm thì ghi vào `deferred-work.md` kèm chủ.

## Conventions that differ from defaults

- Chú thích viết bằng tiếng Việt, dày, và chở LÝ DO — vì sao hình dạng này chứ không phải hình dạng kia, và phương án bị loại đã bị loại bằng gì. Một quyết định không hiển nhiên kèm một PHÉP ĐO (số, ngày, `tệp:dòng`), không một sở thích.
- Mệnh đề hết đúng thì SỬA TẠI CHỖ kèm 🔵 và ngày; đừng xoá, và đừng để nó lặng lẽ sai.
- Ký hiệu: 🔴 luật không được phá · ⚠️ bẫy hoặc giới hạn · ✅ đã đóng · 🟡 đóng một nửa · 🔵 cập nhật, mệnh đề cũ hết đúng · ⇒ kết luận. Emoji `U+26D4` bị cấm toàn kho — viết `không`/`KHÔNG` thành chữ. Đừng đúc một ký hiệu mới cho một quy ước mới; viết thành chữ.
- Commit: `type(scope): câu tiếng Việt`, và câu đó nói ĐIỀU ĐÃ TÌM RA, không chỉ điều đã sửa.
- Thuật ngữ cố định trong mã: Tác phẩm→`Work` · Chương→`Chapter` · Chế độ đọc→`ReadingMode` · Hán Việt→`HanViet` · lớp nền/gỡ rời→`BaseLayer`/`DetachableLayer`. Cấm `Project`/`Book`/`Novel`/`Document` cho `Work`. Miễn trừ đủ tám mục, tất cả đặt tên cho KHO chứ không cho thực thể: `.atproj` · `project.db` · `StoreKind::Project` · `ProjectStore` · `PROJECT_MIGRATIONS` · `commands/project.rs` · `ports/project_store.rs` · `tests/project_contract.rs`. Cổng canh luật này: `src-tauri/tests/naming_boundary.rs` (Story 5.1) — danh sách ở đây và mảng `STORE_EXEMPT` trong cổng phải khớp nhau từng mục.
- Sổ nợ đóng bằng chữ, ba cách: `→ ✅ ĐÃ ĐÓNG <ngày> (Story x.y)` · `→ 🟡` kèm phần còn hở · `→ KHÔNG LÀM <ngày> (Story x.y) — <lý do>`, lý do phải nói ĐIỀU GÌ ĐÃ ĐỔI. Không bao giờ xoá một mục đã đóng.
- Năng lực chưa dựng ≠ lệch spec. Đừng sửa `epics.md`/`prd.md` cho khớp mã đã viết — ghi một món nợ có chủ.

## Known pitfalls

- Rỗng IM LẶNG là lớp lỗi trung tâm của dự án: một truy vấn trả 0 hàng trong 0,01 ms không ném lỗi nào và lộ ra thành *"tra từ không ra kết quả"* mà không ai lần được nguyên nhân. Một danh sách rỗng không tự nói vì sao nó rỗng — hỏi vị từ `…HasLoaded` trước khi kết luận "không có", và kiểm cả THAM SỐ đi vào vị từ đó: ở Story 3.9 vị từ thì đúng, chỗ gọi bịa `totalCount` bằng chính `filteredCount`, nên một bộ lọc quét sạch hàng trên một Glossary CÓ dữ liệu khẳng định "đang trống". Đã hụt BA lần (Story 1.16 · 2.10 · 3.9) và không cổng nào canh.
- Mọi lượt ghi `target_text` mà văn bản KHÔNG đến từ bộ đệm gõ phải đặt CẢ HAI trong cùng thao tác: mốc so sánh VÀ cột xuất xứ (AD-47, danh mục đóng 7 dòng; ngoại lệ có tên duy nhất là khôi phục FR101). Quên vế xuất xứ ⇒ cặp TM mang nhãn sai, không cổng nào đỏ, và lộ ra sau hàng trăm câu dưới dạng *"AI dịch không còn giống giọng tôi"*. Cùng luật, rộng hơn `target_text`: một lượt ghi chạm ĐÚNG những cột người dùng đồng ý đổi — "lấy của file" ở Story 3.10 ghi `SET translation, note, category` vô điều kiện, nên một tệp hai cột xoá sạch ghi chú người dùng tự viết.
- Ranh giới segment tính MỘT LẦN lúc nhập và lưu xuống; không đường mã nào tính lại lúc nạp. Gộp/tách SEGMENT = về hưu + tạo mới, nhưng gộp/tách CHƯƠNG thì KHÔNG (chỉ đổi `chapter_id` và `ord`). Nhầm hai cái này phá sạch lịch sử của những Chương đã dịch xong, vĩnh viễn.
- Sửa KIỂU cho nó nói thật; đừng hạ ngưỡng, thêm `eslint-disable`, hay chuyển một cặp sang danh sách loại trừ để cổng hết đỏ — cả ba đều cho exit 0 trên một sản phẩm đang hỏng. Mọi miễn trừ phải CÓ TÊN, có lý do tại chỗ, và phải chết được.
- Đừng bắt chước một ký hiệu chưa hiểu: `grep` đếm số lần VÀ tìm định nghĩa trước khi dùng lại. Không có định nghĩa ⇒ viết chữ thường minh và nêu với Ice kèm số đo.
- Một bộ test xanh KHÔNG chứng minh chỗ nối mới được canh — Epic 3 dính NĂM lần trong BẢY ngày: 0/12 spec e2e chạm bề mặt mà Story 3.4b và 3.5 dựng; 68 ca Rust xanh trong khi `work_context` thoái hoá thành luôn trả `None`; 412 dòng test của dải chưa mount component lần nào; và ở Story 3.10 gỡ bản vá ra thì hai ca MỚI đỏ còn ca CŨ `..._take_theirs_updates_the_existing_row` vẫn xanh. Trước khi khai một mệnh đề là đạt, `grep` đếm số ca THẬT SỰ chạm bề mặt đó, và đối chứng bằng cách GỠ chỗ nối rồi chạy bộ test CŨ — nó phải đỏ.

<!-- /bmad:context -->
