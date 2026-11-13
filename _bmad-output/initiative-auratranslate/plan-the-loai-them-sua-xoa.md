---
title: 'Thể loại: thêm, sửa, xoá trong danh sách'
type: 'feature'
ticket: ''
created: '2026-11-13'
status: 'built'
baseline_revision: 'b76ca6ac0ddc6d5d4e56c0505142733c7fce2786'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Danh sách Thể loại là 29 mục cố định trong mã (`GENRE_IDS`, nhãn ở `vi.json`). Ice chỉ chọn được trong danh sách, không thêm, đổi tên hay xoá được.

**Approach:** Danh sách chuyển vào một bảng của `global.db`, và Ice thêm, đổi tên, xoá trong một mục "Thể loại" mới của Cài đặt. Ô Thể loại ở hai màn nhập, chip lọc "Lĩnh vực", thẻ và trang Tác phẩm, "Thêm từ mẫu" và tự chọn bộ prompt đều đọc danh sách này.

**Đã chốt (Ice, 2026-11-13):**
- Cả 29 mục có sẵn cũng đổi tên và xoá được, cùng luật với mục tự thêm.
- Bảng `genre` mới ở `global.db`, bước 18 của `GLOBAL_MIGRATIONS`, với các lệnh list/create/rename/delete theo khuôn `prompt_set`. Rust từ chối tên rỗng và tên trùng (AD-1).
- `work.genre` lưu định danh ổn định. 29 mục có sẵn giữ định danh cũ (`xianxia`, `news`…). Mục mới nhận định danh sinh ra, không dùng lại sau khi xoá. Đổi tên thì thẻ, chip và menu hiện tên mới ngay. Không xoá được một thể loại còn Tác phẩm dùng, và app báo số Tác phẩm đó.
- Bước 18 gieo 29 hàng theo đúng thứ tự menu hôm nay. Tên đọc từ một tệp dữ liệu đóng vào bản dựng, theo khuôn `default_system_prompt.txt`. 29 khoá `mode.library.genre_<id>` rời `vi.json`, còn `genre_unset` ở lại.
- Giữ một plan dù vượt 1600 token. Plan 1117 từ, chưa đếm được bằng tiktoken vì máy không có gói này.

## Boundaries & Constraints

**Always:** Rust so tên sau khi trim, không phân biệt hoa thường theo Unicode (`Đ`/`đ`). "Chưa chọn" không phải hàng của bảng: luôn đứng đầu menu, không sửa và không xoá được. Giá trị `genre` rỗng thì không hiện, giá trị lạ (`'Huyền huyễn'`, `'general'`) hiện nguyên văn. Mục mới vào cuối danh sách. Lỗi đi qua `tError`.

**Never:** Không làm việc đổi Thể loại cho Tác phẩm có sẵn (đã có nợ riêng trong `deferred-work.md`). Không tự tạo hay tự đổi tên bộ prompt khi thể loại đổi. Không sửa thân 29 mẫu prompt. Không sắp xếp lại thứ tự danh sách.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Di trú | `global.db` ở bước 17 | 29 hàng, đúng thứ tự và tên của menu hôm nay | — |
| Thêm | "Truyện ngắn" | Nằm cuối menu ở cả hai màn nhập | — |
| Trùng | "tiên hiệp", hoặc "đô thị" khi đã có "Đô thị" | Không thêm, không đổi tên | Báo trùng tên |
| Rỗng | "   " | Không thêm | Báo tên rỗng |
| Đổi tên có sẵn | `news` "Thời sự" → "Tin tức" | 2 Tác phẩm `news` và chip lọc hiện "Tin tức". Tự chọn khớp bộ "Tin tức" | — |
| Xoá đang dùng | Xoá `news` (2 Tác phẩm) | Không xoá | Báo "đang dùng ở 2 Tác phẩm" |
| Xoá không dùng | Xoá `wuxia` | Biến mất khỏi menu; "Thêm từ mẫu" không còn mẫu Kiếm hiệp | — |
| Tự thêm, tự chọn | Tác phẩm mang thể loại "Truyện ngắn", có bộ "Truyện ngắn" | Bộ đó được tự chọn khi mở Tác phẩm | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/store/schema.rs:109,800-883` -- `Migration { to_version, sql: &'static str }`; `GLOBAL_MIGRATIONS` dừng ở 17 (các bước 18+ ở dòng 2474 thuộc `project.db`). Gieo bằng `concat!(GENRE_DDL, include_str!("…"))`, vì Kiểm A của `check:i18n` cấm chữ tiếng Việt trong `.rs`.
- `src-tauri/src/core/store/schema.rs:1432` `PROMPT_SET_DDL` (`UNIQUE` + `CHECK` trim), `src-tauri/src/core/promptset/store.rs:50,210,268,359` (`validate_name`/`create`/`rename`/`delete`, bắt `is_unique_constraint_violation`) -- khuôn. `UNIQUE` của SQLite chỉ gập hoa thường ASCII, nên Rust phải tự so trước.
- `src-tauri/src/commands/promptset.rs:788-950` -- khuôn vỏ lệnh. `src-tauri/src/core/i18n/mod.rs` -- `message_key` mới; `tests/ipc_contract.rs` kiểm khoá có trong `vi.json`.
- `src-tauri/src/core/library/indexer.rs:742` -- `library_work.genre` trong `library-index.db`. Phép đếm "đang dùng" đọc từ đây.
- `src/modes/libraryWorkCard.ts:28-76` -- `GENRE_IDS` (vẫn là kiểu khoá của `PROMPT_TEMPLATES`), `genreLabel` (đồng bộ, 9 chỗ gọi), `genreChoices`. Cả hai hàm đọc một ref danh sách, nạp ở `src/main.ts` trước `mount()` và nạp lại sau mỗi lần sửa.
- `src/LibraryImportOverlay.vue:49`, `src/modes/LibraryUrlImportScreen.vue:47`, `src/modes/LibraryMode.vue:258,512` -- đã đi qua `genreChoices`/`genreLabel`, không cần sửa.
- `src/promptSetState.ts:318,371-398`, `src/PromptTemplatePicker.vue:22` -- tên mẫu lấy qua `genreLabel(id)`; picker chỉ liệt kê id còn trong bảng.
- `src/SettingsOverlay.vue:67-75,407`, `src/settingsState.ts` (`SETTINGS_SECTIONS`) -- thêm mục. Hàng có nút theo khuôn `src/SettingsShortcutsSection.vue:250-277`: chọn hàng, rồi `@click` là đúng một `dispatch('<id>')`.
- `src-tauri/src/commands/project/work_creation.rs:83,455` -- `genre` đi thẳng xuống, không kiểm; không đổi.
- Dữ liệu thật hôm nay: 49 Tác phẩm (47 rỗng, 2 `news`); 29 bộ prompt Toàn cục mang đúng 29 tên.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/genre/` (mới), `src-tauri/src/core/store/schema.rs` -- DDL, tệp gieo, bước 18, luật tên, list/create/rename/delete. `delete` từ chối khi số Tác phẩm đang dùng > 0.
- [ ] `src-tauri/src/commands/genre.rs` (mới), `src-tauri/src/lib.rs`, `src-tauri/src/core/i18n/mod.rs` -- bốn lệnh và khoá lỗi `genre.name_empty`/`name_taken`/`in_use` (`params.count`).
- [ ] `src/config/genre.ts` (mới), `src/genreState.ts` (mới), `src/main.ts`, `src/modes/libraryWorkCard.ts`, `src/PromptTemplatePicker.vue` -- adapter, nạp lúc khởi động, nhãn từ bảng.
- [ ] `src/SettingsGenreSection.vue` (mới), `src/SettingsOverlay.vue`, `src/settingsState.ts`, `src/main.ts` (đăng ký `settings.genre.*`), `src/i18n/vi.json` -- mục Cài đặt; bỏ 29 khoá `genre_<id>`.
- [ ] `src-tauri/tests/genre_contract.rs` (mới), `tests/frontend/libraryWorkCard.test.ts`, `tests/frontend/settingsGenreSection.test.ts` (mới), các fixture đang đọc `mode.library.genre_*` -- phủ mọi hàng Matrix.

**Acceptance Criteria:**
- Given Cài đặt, when mở mục Thể loại, then thấy 29 mục theo thứ tự menu hôm nay, và thêm, đổi tên, xoá được bằng cả chuột lẫn bàn phím.
- Given đổi "Tiên hiệp" thành "Tu tiên" rồi khởi động lại app, when mở menu Thể loại ở màn nhập, then "Tu tiên" đứng đúng chỗ cũ.

## Implementation Notes

- DDL và 29 hàng gieo nằm chung trong `src-tauri/src/core/genre/genre_schema.sql`, vì `concat!` không nhận `const`; `GENRE_DDL` là `include_str!` của tệp đó.
- Định danh mới là `g<seq>` lấy từ `sqlite_sequence`; 29 hàng gieo đặt `seq` 1–29 tường minh, nên mục đầu tiên Ice thêm là `g30`.
- `genre_delete` không có chỉ mục Library thì trả `library.indexer_missing`, không coi là 0 Tác phẩm; `indexer_is_missing` thành `pub(crate)`.
- Bước 18 làm cũ các test ghim số phiên bản 17: `store_contract`, `pinned_contract`, `tm_contract`, `glossary_*`, `prompt_set_kind_migration_contract`, cộng tổng lệnh trong `config_invariants`. Lượt `cargo test` trọn bộ bắt được thêm `store_contract` sau khi agent khai đã xanh.
- Hai ca tự chọn bộ prompt (thể loại tự thêm, thể loại đổi tên) được đối chứng bằng cách gỡ chỗ `genreLabel` đọc bảng: cả hai đỏ, trả lại thì xanh.
- Lượt chạy trọn bộ: vitest 4455/4456, `cargo test` chỉ đỏ `store_contract`, đã vá. Ca vitest đỏ còn lại (`exportScope`, 8 ô chọn thay vì 7) và `check:panel-refs` (`lookupPaint.ts::paintedBlocks`) đỏ sẵn từ `66a400ab` và `b76ca6ac`, diff này không chạm.

## Plan Change Log

## Review Triage Log

Quick, vòng 1: high 0 · medium 0 · low 7 · false 0 · maybe-false 0. Vá 3, bác 4.

- low · patch — hàng Matrix "Di trú từ bước 17" chỉ có ca kho mới; các test di trú khác chỉ đổi số 17→18, không đếm 29 hàng.
- low · patch — comment ba dòng đầu `SettingsGenreSection.vue` kể lại template (AGENTS.md §Code comments).
- low · patch — đường bàn phím `@focusin` và ca đổi tên bị Rust từ chối chưa có test ở `settingsGenreSection.test.ts`.
- low · reject — lỗi `genre_list` lúc khởi động chỉ ghi console (đã thêm). `genre_list` đọc cùng `global.db` với `bootstrap_config`, nên kho hỏng thì app đã hỏng trước đó; mục Cài đặt nạp lại và hiện `tError` khi mở. Báo ở hai màn nhập là thêm giao diện mới.
- low · reject — Tác phẩm chưa vào chỉ mục không được đếm khi xoá, nên sau khi xoá sẽ hiện mã thô. Plan đã chọn đếm theo `library-index.db`, và chỉ mục được dựng lại mỗi lần quét. Muốn sửa phải đọc mọi `meta.json`.
- low · reject — kiểm trùng tên nằm ngoài giao dịch ghi. Cờ `genreBusy` tuần tự hoá các thao tác trong app một người dùng, nên hai lệnh tạo không đua nhau được.
- low · reject — chuyển sang hàng khác thì mất bản nháp đổi tên của hàng cũ. Rời hàng đọc được là bỏ sửa; giữ nháp theo từng hàng là thêm trạng thái mới.

## Design Notes

AD-21 cấm chuỗi tiếng Việt trong *mã* Rust. Tên gieo nằm trong tệp dữ liệu (tiền lệ `default_system_prompt.txt`) và từ lúc gieo trở thành dữ liệu của Ice như tên bộ prompt, nên không cần AD mới.

## Verification

**Commands:**
- `npm run build`, rồi `cargo test --test genre_contract` -- xanh.
- `npx vitest run tests/frontend/libraryWorkCard.test.ts tests/frontend/settingsGenreSection.test.ts tests/frontend/promptSetState.test.ts` -- xanh.
- `npm run check:i18n`, `npm run check:commands`, typecheck -- xanh.
- Chạy trọn bộ test một lần, vì đổi migration và `lib.rs` (AGENTS.md §Tests).

**Manual checks (if no CLI):**
- Ice, trong app thật: đổi tên một thể loại, khởi động lại, xem thẻ và menu; xoá `news` thì bị chặn.
