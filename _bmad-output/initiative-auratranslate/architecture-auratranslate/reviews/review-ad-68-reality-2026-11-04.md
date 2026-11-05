# Review AD-68 theo thực tế mã tại HEAD (fb652559) — 2026-11-04

Phạm vi: chỉ đối chiếu các khẳng định về mã và công cụ trong `### AD-68`. Không sửa spine hay mã. Đường dẫn tính từ `src-tauri/` nếu không ghi khác.

## Kết luận

AD-68 đứng được về thiết kế, nhưng có MỘT khẳng định sai về công cụ (phiên bản SQLite), MỘT khẳng định sai về phạm vi helper dùng lại (U+200B không phủ dấu `SOURCE`/`TEXT`), MỘT test canh (g) đo nhầm đối tượng, và thiếu danh sách các test/gate hiện có sẽ đỏ khi áp AD này.

## Bảng khẳng định

| # | Khẳng định | Phán | Bằng chứng |
|---|---|---|---|
| 1 | `assemble_proofread_batch_prompt` là hàm thuần, thứ tự hiện nay: giới thiệu → FORMAT → luật loại → QUOTE → EXPLANATION → khối | ĐÚNG | `src/core/ai/rag.rs:965-993`; hằng ở `931-952`; câu `format!` ở `975-977` |
| 2 | Hằng `PROOFREAD_INSTRUCTION` / `_WITH_SOURCE`, `PROOFREAD_FORMAT`, `PROOFREAD_QUOTE_RULE`, `PROOFREAD_EXPLANATION_RULE` tồn tại; luật loại có hai bản (`PROOFREAD_KIND_RULE`, `_WITH_SOURCE`) | ĐÚNG | `rag.rs:931-944` |
| 3 | Rule 4: chèn chỉ dẫn giữa câu giới thiệu và `PROOFREAD_FORMAT` mà không có chỉ dẫn thì trùng từng byte | ĐÚNG, làm được | Hiện `{instruction}\n{FORMAT}...` (`rag.rs:975-977`); chỉ cần nhánh `Some` chèn `\n{guide}` |
| 4 | Có helper chèn U+200B của AD-60 mục 4 mà đường proofread dùng lại được | ĐÚNG MỘT PHẦN | `core/ai/batch.rs:31` (`ZERO_WIDTH_SPACE`), `104-113` (`guard_marker_line`), `129-134` (`guard_text`); `rag.rs:41,863` đã `use` và gọi `guard_text` ở đường dịch. Xem Phát hiện 2 |
| 5 | Dấu khối của lượt quét "đã được chèn U+200B" hôm nay | SAI (ngược lại) | Đường quét KHÔNG gọi `guard_text`: nguồn và bản dịch được ghi nguyên văn ở `rag.rs:981-987`. Chỉ đường dịch được canh (`rag.rs:863`) |
| 6 | "Một dòng chỉ dẫn trùng một dấu khối của lượt quét" được `guard_marker_line` bắt | SAI MỘT PHẦN | `classify_line` (`batch.rs:51-70`) chỉ nhận `<<<SEGMENT n`, `SEGMENT>>>`, `<<<CONTEXT`, `CONTEXT>>>`, `<<<TM`, `TM>>>`. `<<<SOURCE`, `SOURCE>>>`, `<<<TEXT`, `TEXT>>>` (`rag.rs:946-952`) KHÔNG nằm trong tập, nên helper để lọt đúng các dấu lượt quét thêm vào |
| 7 | `commands/proofread.rs` gửi `system: None`; plan lô bằng `plan_batches` chỉ đếm nguồn + bản dịch | ĐÚNG | `commands/proofread.rs:202` và `295` (`system: None`); `183` (`plan_batches(&sizes)`), `sizes` ở `178-181` chỉ cộng `source_text` + `target_text` |
| 8 | Chỗ chèn "đọc lựa chọn một lần mỗi lượt quét" hợp lý | ĐÚNG | `prepare_proofread_call(global, open, segment_ids)` (`proofread.rs:137-141`) có cả `global` và `open` (hai kho), chạy một lần trước vòng lô (`proofread.rs:178-215`) |
| 9 | `PROMPT_SET_DDL` có bảng ở cả `global.db` và `project.db`, chỉ mục UNIQUE trên `name` | ĐÚNG | `schema.rs:1373-1388` (`CREATE UNIQUE INDEX idx_prompt_set_name ON prompt_set (name)`); `GLOBAL_MIGRATIONS` bước 9 (`schema.rs:~845-850`); chú thích ghi cùng hằng ở bước 24 của `project.db` |
| 10 | `CONFIG_VALUE_DDL` là `(kind, key, value)`; `app_config` là loại `GlobalOnly` | ĐÚNG | `schema.rs:142-149`; `core/scope/kinds.rs:234` (`AppConfig => "app_config" : GlobalOnly`) |
| 11 | Một hàng `config_value` `app_config` / `proofread_prompt_set` đủ để lưu | ĐÚNG MỘT PHẦN | Bảng không cần đổi (`CONFIG_VALUE_DDL` không `CHECK` liệt `key`). Nhưng khoá được liệt kê trong mã: mỗi khoá là một hằng riêng trong `core/scope/store.rs:49-188` (`KEY_THEME`, `KEY_UPDATE_CHECK_ENABLED`...) kèm bộ truy cập kiểu; AD-68 phải nêu thêm `KEY_PROOFREAD_PROMPT_SET` và đường ghi/đọc. Xem Phát hiện 5 |
| 12 | `work.chosen_prompt_set_name` có sẵn | ĐÚNG | `schema.rs:1117-1118`, bước 41 (`schema.rs:2526`); `core/promptset/store.rs:356-380` (`read_chosen_name`/`write_chosen_name`) |
| 13 | Cột mới `work.chosen_proofread_prompt_set_name` không va chạm cột cũ | ĐÚNG | Tên khác; nhưng cột cũ mang ngữ nghĩa "chọn tay, nếu không thì Thể loại (Genre) tự chọn" (`schema.rs:1115-1116`, `src/promptSetState.ts:20-27`). Proofread không có luật Thể loại; AD nên nói rõ |
| 14 | Danh sách bước di trú: `GLOBAL_MIGRATIONS` (đích 15), `PROJECT_MIGRATIONS` (đích 42) | ĐÚNG | `schema.rs:800-` (bước cuối 15, duyệt bằng awk); `schema.rs:2526-2533` (41, 42). Bước mới là 16 và 43 |
| 15 | Một bước di trú gồm `ALTER TABLE ADD COLUMN` + `DROP INDEX` + `CREATE UNIQUE INDEX` được | ĐÚNG | `migrate` chạy `tx.execute_batch(m.sql)` trong một giao dịch (`schema.rs:2954-2990`), nhiều câu được. Tiền lệ: `PROMPT_SET_DDL` đã là đa câu (`CREATE TABLE` + `CREATE UNIQUE INDEX`) |
| 16 | "SQLite 3.43.2" | SAI | `libsqlite3-sys 0.38.1` (`Cargo.lock:2676-2677`, `rusqlite =0.40.1 bundled` ở `Cargo.toml:37`) đóng gói `SQLITE_VERSION "3.53.2"` (`~/.cargo/registry/src/*/libsqlite3-sys-0.38.1/sqlite3/sqlite3.h`). Kết luận về hình dạng di trú vẫn đúng ở 3.53.2 (`ADD COLUMN ... NOT NULL DEFAULT ... CHECK` được phép và kiểm hàng có sẵn từ 3.37) |
| 17 | Chỉ mục duy nhất trên `(kind, name)` nhận hai bộ cùng tên khác loại | ĐÚNG (chưa có test) | Theo ngữ nghĩa SQLite; AD nên ghi test (e) thành một ca tầng SQL chứ không chỉ tầng Rust |
| 18 | `resolve_two_tiers` phân giải theo tên, ghi đè cả bộ | ĐÚNG | `core/promptset/store.rs:115-160`; nhưng `load_prompt_set_tier` trả `BTreeMap<String, PromptSet>` khoá theo `name` (`store.rs:63-77`) và `SELECT id, name, body FROM prompt_set` không lọc loại (`store.rs:65`). Sau AD này hai hàng cùng tên khác loại cùng tầng sẽ ĐÈ nhau trong map nếu không lọc `kind` ngay trong truy vấn. Rule 1 đã bảo "mọi đường đọc nhận loại"; nên đặt tên chỗ này |
| 19 | `exchange.rs` có `render`/`parse` đúng ba dòng cố định | ĐÚNG | `exchange.rs:56-58` (`render`), `184-221` (`parse`, `line3 != DELIMITER` → `MissingClosingDelimiter`) |
| 20 | Bản app cũ từ chối tệp `kind: proofread` bằng `MissingClosingDelimiter` | ĐÚNG | Dòng 3 = `kind: proofread` ≠ `---` ⇒ `exchange.rs:208-210` |
| 21 | `ParseIssue`, `classify`, `import_into_tier` cần nhận loại | CHƯA NÊU trong AD | `ParsedPromptSet` chỉ có `name`+`body` (`exchange.rs:215-218`); `classify` nhận `BTreeMap<String, PromptSet>` (`exchange.rs:260`); cần thêm `kind` vào cả hai và một biến thể lỗi cho `kind:` lạ cùng khoá i18n `MessageKey` mới |
| 22 | `prepare_operation_prompt` có `no_set_selected` | ĐÚNG | `commands/aiprompt.rs:68-75`, dùng tại `761` và `764`. Lưu ý: `prompt_set_name` là THAM SỐ do webview truyền (`aiprompt.rs:748-752`; `ipc_contract.rs:2371,2560,2589`) |
| 23 | `no_set_selected` "nêu tên" | SAI cho mẫu muốn noi theo | Khoá `err.ai_prompt.no_set_selected` không có tham số (`core/i18n/mod.rs:848`, `[]`), không nêu tên. Lỗi mới của proofread phải khai `params = ["name"]` và thêm vào `vi.json` |
| 24 | Webview đường dịch truyền tên bộ | ĐÚNG | `src/config/aitranslate.ts:127,292`; `src/aiTranslateBatchState.ts:229-264`; `src/config/aiprompt.ts:338`; chọn tay lưu qua `prompt_set_choice_*` (`src/promptSetState.ts:20-27`) |
| 25 | "Rust tự đọc, webview không truyền tên" cho proofread | KHẢ THI, KHÔNG có tiền lệ | Hôm nay không command nào bên đường AI tự đọc lựa chọn bộ prompt. Prepare nhận `global`+`open`, nên đọc được cả hai tầng; AD nên nói đây là ngoại lệ có chủ ý so với đường dịch và thêm lệnh IPC đọc/ghi lựa chọn proofread cho màn Cài đặt |
| 26 | Cổng `ai_boundary` bị đụng bởi `commands/proofread.rs` | CHƯA KIỂM ĐƯỢC chắc | `ALLOWED_AI_PATHS_IN_PROOFREAD_SEAM` (12 mục, `tests/ai_boundary.rs:642-655`) chỉ liệt đường `crate::core::ai::`. Gọi `crate::core::promptset::` và `crate::core::scope::` không nằm trong cổng đó, nhưng cổng lib.rs của `commands::aiprompt` (`ai_boundary.rs:105-130`) là loại cổng nhạy với đăng ký lệnh mới. Cần chạy `cargo test --test ai_boundary` sau khi đổi |
| 27 | Màn nhập/thư viện hiện tại biết "loại" | SAI (chưa có) | `src/PromptLibraryOverlay.vue`, `src/PromptImportOverlay.vue`, `src/promptSetState.ts` chưa có khái niệm loại; `grep -i proofread` trên `PromptLibraryOverlay.vue` không ra gì. Đây là việc mới đúng như AD nói |

## Phát hiện chính

1. **SQLite 3.43.2 là sai; bản đóng gói là 3.53.2** (`libsqlite3-sys 0.38.1`). Kết luận không đổi nhưng dòng bằng chứng cuối AD-68 phải sửa, vì quy tắc dự án cấm số đo không nêu build.
2. **U+200B dùng lại được nhưng KHÔNG phủ dấu của lượt quét.** `classify_line` chỉ biết SEGMENT/CONTEXT/TM. `<<<SOURCE`/`SOURCE>>>`/`<<<TEXT`/`TEXT>>>` không được canh, và nguồn/bản dịch trong đường quét hiện không đi qua `guard_text` (`rag.rs:981-987`). Muốn "chỉ dẫn trùng dấu khối của lượt quét thì được canh", phải mở rộng `classify_line`/`could_become_marker` hoặc viết hàm canh riêng, mà `classify_line` còn phục vụ bộ đọc dịch (đổi nó đổi cả đường dịch, ai_batch_wire).
3. **Test canh (g) đo nhầm đối tượng.** "Bộ đọc vẫn ra đúng số khối" không áp dụng: đường quét không có bộ đọc khối; `locate_batch_findings` đọc một mảng JSON (`core/ai/proofread.rs:109`). Test (g) phải khẳng định trên byte của prompt: dòng trùng dấu có U+200B ở vị trí thứ hai, và số dòng-dấu thật trong prompt đúng số khối.
4. **Test hiện có sẽ đỏ khi áp AD** (phải cập nhật cùng lượt): `schema_version() == 42` ở `tests/prompt_set_contract.rs:895`, `tests/tm_contract.rs:823`, `tests/review_harvest_contract.rs:701` (thành 43); `== 15` ở `tests/pinned_contract.rs:163,698`, `tests/glossary_contract.rs:3445` (thành 16, nếu các ca này mở `global.db`); `tests/prompt_set_contract.rs:880-896` (hạ bước 40 rồi kiểm 42) và các ca `DELETE FROM schema_migration_log` tương tự. Các lệnh IPC mới làm lệch bảng đếm `61 plain / 41 async` ở `tests/ipc_contract.rs:2159` và `tests/config_invariants.rs:~2112`, cùng danh sách `the_prompt_set_wires_are_registered` (`ipc_contract.rs:2289-2309`). `tests/prompt_set_exchange_contract.rs` (ba dòng cố định, `:609`, `:745`) cần ca bổ sung chứ không đỏ, miễn đường ba dòng giữ nguyên.
5. **Khoá `app_config` không tự do.** Mỗi khoá là một hằng riêng và có bộ truy cập kiểu trong `core/scope/store.rs:49-188`; không có bảng tổng. AD-68 nên nêu rõ: thêm `KEY_PROOFREAD_PROMPT_SET`, bộ đọc/ghi, lệnh IPC, và (giống `tm_fuzzy_threshold`, `store.rs:137-160`) định nghĩa ý nghĩa của giá trị hỏng/rỗng trên đĩa, để "không hàng" và "chuỗi rỗng" không bị lẫn thành một.
6. **Tên "như `no_set_selected`" dễ gây hiểu lầm.** Khoá đó không nêu tên và gộp hai nguyên nhân (không chọn / tên không khớp) làm một (`aiprompt.rs:64-75`). Với proofread, "không chọn" là hợp lệ (không chỉ dẫn), còn "tên không còn" là lỗi; hai ca này đã khác nhau trong AD, nên khoá lỗi phải mới, có tham số `name`.

## Đề xuất sửa chữ AD-68

1. Bằng chứng cuối, dòng SQLite: thay "SQLite 3.43.2" bằng "SQLite 3.53.2 (`libsqlite3-sys 0.38.1`, `rusqlite =0.40.1 bundled`)".
2. Rule 4, gạch đầu dòng U+200B: thay bằng
   - "Một dòng của chỉ dẫn trùng một dấu khối được chèn U+200B vào giữa dấu. Tập dấu cần canh cho lượt quét gồm `<<<SEGMENT n`, `SEGMENT>>>` (đã có ở `core/ai/batch.rs`) và `<<<SOURCE`, `SOURCE>>>`, `<<<TEXT`, `TEXT>>>` (chưa có: `classify_line` chưa biết chúng). Việc mở rộng `classify_line` phải giữ nguyên byte của mọi prompt dịch."
   - Hoặc, nếu không muốn đụng bộ đọc dịch: "hàm canh riêng của `rag.rs` cho sáu dấu của lượt quét; chỉ `guard_marker_line` cho dấu SEGMENT được dùng lại".
   - Thêm câu: "Nguồn và bản dịch trong khối KHÔNG được canh trong AD này (hôm nay cũng không), để prompt không đổi byte khi không có chỉ dẫn." Nếu Ice muốn canh luôn, đó là AD khác vì làm đổi byte của prompt hiện có.
3. Test (g): thay vế sau bằng "và số dòng là dấu khối thật trong prompt đúng bằng số khối của lô". Bỏ cụm "bộ đọc".
4. Rule 3, lỗi tên mất: ghi "khoá lỗi i18n mới (`MessageKey` + `vi.json`) mang tham số `name`; KHÔNG dùng lại `AiPromptNoSetSelected`, khoá đó không có tham số".
5. Rule 3, tầng Global: thêm "hằng khoá và bộ đọc/ghi kiểu nằm ở `core/scope/store.rs` cùng nhóm với các khoá `app_config` khác; hai lệnh IPC `prompt_set_proofread_choice_get/set` (tên chốt khi viết story) đăng ký ở `lib.rs` và tính vào bảng đếm lệnh".
6. Rule 1, thêm "`load_prompt_set_tier` lọc `WHERE kind = ?` trong truy vấn; `ParsedPromptSet`, `PromptSet` và `classify` mang `kind`".
7. Rule 3, thêm một câu: "Lựa chọn bộ proofread không có luật Thể loại tự chọn của đường dịch (`promptSetState.ts`); không tự chọn là không chỉ dẫn".
8. Rule 8, thêm mục "Test hiện có phải cập nhật cùng lượt": các test số phiên bản (`prompt_set_contract`, `tm_contract`, `review_harvest_contract` 42→43; `pinned_contract`, `glossary_contract` 15→16) và bảng đếm lệnh ở `ipc_contract`/`config_invariants`.
9. Rule 7 (c)/(d): nêu đây là ca ở `commands/proofread.rs::prepare_proofread_call`, vì chỗ đó (và chỉ chỗ đó) có cả `global` lẫn `open`.

## Chưa kiểm được

- Mục 26: chưa chạy `ai_boundary`; chỉ đọc nguồn. Kết luận phải đo bằng `cargo test --test ai_boundary` sau khi đổi mã.
- Mục 17: chưa chạy SQL mẫu trên SQLite 3.53.2; dựa trên ngữ nghĩa đã biết của `CREATE UNIQUE INDEX` hai cột.
- Hai hằng `== 15` ở `pinned_contract`/`glossary_contract`: tôi suy ra là `global.db` từ tên và chú thích ("buoc 3 den 15", "buoc 5 den 15"); chưa mở từng ca xác nhận kho nào được mở.
