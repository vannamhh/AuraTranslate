---
title: 'Story 3.4 — Khớp thuật ngữ theo ngôn ngữ qua Matcher dùng chung'
type: 'feature'
created: '2026-08-21'
status: 'done'
baseline_commit: '0c5bf3dcb2cc511b21bb21191fdab24051b31d0d'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-3-them-nhanh-thuat-ngu-tu-bat-ky-panel-nao.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `core/matching/mod.rs` được Story 1.12 dựng **đúng cho story này** — doc-comment nêu đích danh FR51/Story 3.4 — và tới hôm nay có **0 người tiêu thụ**, nên hình dạng API của nó vẫn là *"một phỏng đoán có căn cứ"* (lời của chính module). Glossary sau 3.3 thêm được mục nhưng không đường nào **hỏi** *"câu này chứa thuật ngữ nào"*.

**Approach:** Dựng nửa **Rust** của FR50/FR51: một hàm phơi ra **thứ tư** của `core::glossary` tra hai tầng rồi gọi `find_terms`, cộng một bề mặt IPC hai lớp trả **dấu theo từng segment**. Nửa giao diện (vẽ dấu, `StatusBar`) tách ra — Ice ký 2026-08-21, chủ ghi trong sổ nợ.

## Boundaries & Constraints

**Always:**
- 🔴 **Dùng `core::matching::find_terms`, không cài lại phép khớp** (AD-17). `MatchLang` đến từ `work.source_lang` (`"zh"` ⇒ `Zh`, mọi giá trị khác ⇒ `En` — cùng nhánh `split.rs:219`), **không** đoán từ nội dung.
- 🔴 **Đơn vị trên dây là ĐIỂM MÃ.** `find_terms` trả **byte**; cả lưới (`sourcePieceStartsOf` · `sourceCutOffsetOf` · `regroup::split_at`) đã dùng **điểm mã**; chuỗi JS lại là **UTF-16**. Quy đổi làm **trong Rust**, một lần, một chỗ.
- 🔴 **Hâm nóng `Jieba` NGOÀI đường gõ** — ở đường **mở Chương**, thứ đã tồn tại (`commands/chapter.rs:119`). Đo (`deferred-work.md:413`): khởi tạo lạnh **179–329 ms** bản release, trung vị ~243 ms, vượt trần NFR2 (50 ms) **3,6–6,6×**; lượt ấm kế tiếp 1 µs.
- 🔴 **Ba tên trong `GLOSSARY_ONLY_SURFACE` ở lại bị cấm ngoài `core/glossary/**`.** Bề mặt IPC gọi hàm phơi ra **MỚI** — tiền lệ có chữ ký của Ice hai lần (3.1, 3.3): sửa **chữ ký**, không nới cổng.
- Lượt tra **không lọc `is_confirmed`** — mục chờ chốt phải ra được, mang cờ phân biệt. `entries_eligible_for_injection` lọc, nên nó **không** dùng lại được ở đây.
- Trùng thuật ngữ hai tầng: phân giải qua `ScopeResolver::apply_override` (AD-18, tầng Tác phẩm thắng) **trước** khi khớp.
- Khuôn hai lớp: hàm thuần nhận `Option<&Store>`; `mod wire` mỏng dùng **`try_state`**, không `state()`.
- Chuỗi literal trong `src-tauri/src/**` viết **không dấu**; mọi chữ hiển thị là khoá `vi.json`.

**Ask First:**
- Nếu phải thêm bất kỳ phụ thuộc nào (cửa NFR15).
- Nếu phép đo `find_terms` trên một Glossary vài nghìn mục cho thấy cần chỉ mục ngược hoặc cache (`deferred-work.md:243`, `:424`) — **đo trước, chốt sau**.
- Nếu phải **hạ** một sàn quần thể thay vì nâng.

**Never:**
- Vẽ một pixel nào: dấu ở cột nguyên văn · cắt `.hv-unit` · dòng `StatusBar` — cả ba thuộc nửa giao diện đã tách (sổ nợ, chủ: Ice qua `correct-course`).
- Chạm `candidate_store` · 3.6 · 3.7 · 3.8 · 3.9.
- Thêm một khoá `3-4b` thẳng vào `sprint-status.yaml` — mọi story hậu tố `b` của kho đi qua `correct-course`.
- Sửa `epics.md`/`prd.md` cho khớp mã.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Trung, khớp chính xác | `lang='zh'`, thuật ngữ `中國`, câu `中國人` | Một dấu phủ đúng `中國` | — |
| Cắt ngang một từ jieba | thuật ngữ `文`, câu `文化` | **Không** dấu nào | — |
| Anh, biến thể hình thái | thuật ngữ `run`, câu `…running…` | Một dấu phủ `running` | — |
| Anh, cực cấp | thuật ngữ `happy`, câu `…happiest…` | **Không** dấu — giới hạn Porter2 đã ký | — |
| Mục chờ chốt | `translation IS NULL` | Có dấu, `is_confirmed=false`, `translation=null` | — |
| Trùng hai tầng | cùng `source_term` ở `global.db` và `project.db` | Đúng **một** dấu, mục **tầng Tác phẩm** (AD-18) | — |
| Chưa mở Tác phẩm | `OpenWorkState` là `None` | Chỉ khớp tầng Global — không lỗi | — |
| Chồng nhau | `AA` và `AAA` cùng khớp một chỗ | Đúng **một** dấu: span **dài nhất**, hoà thì **trái nhất** | — |
| Ký tự ngoài BMP | thuật ngữ chứa `𠧜` | Span điểm mã đúng, không lệch, không panic | — |
| Glossary rỗng | không mục nào | `[]`, và nó **phân biệt được** với lượt tra trượt | — |
| `Store` đóng giữa chừng | kho không mở được | Lỗi mang `message_key`, **không** `Ok(vec![])` | `GlossaryError::Store` |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/matching/mod.rs` — `find_terms` `:455` (**điểm vào**, span **byte** vào chuỗi gốc, sắp tất định theo `(span.start, span.end, term_index)`, trả span **chồng nhau được**) · `MatchLang` `:173` · `TermMatch` `:200` (`term_index` trỏ vào lát `terms` của **chỗ gọi**, không phải id Glossary) · `HMM = false` kèm bảng đo phồn thể. `Zh` lọc theo ranh giới token jieba; `En` chuẩn hoá **cả hai vế** và từ chối dãy bắc cầu qua `.`/`!`/`?`/`\n`.
- `src-tauri/src/core/glossary/store.rs` — `load_tier` `:232` (trả **cả** mục chờ chốt — **bị cấm** ngoài module) · `entries_eligible_for_injection` `:436` (🔴 **lọc `is_confirmed`** ⇒ không dùng lại được) · `resolve_term_for_quick_add` `:509` (**khuôn để chép**: tra hai tầng qua `apply_override`, không lọc `is_confirmed`) · `GlossaryError` `:328` + `impl From<GlossaryError> for IpcError` `:~378`.
- `src-tauri/src/core/glossary/entry.rs` — `GlossaryEntry` `:195` · `is_confirmed` `:215` · `GlossaryTier` `:144` (khuôn enum + `as_str`/`from_wire`).
- `src-tauri/src/commands/glossary.rs` — bề mặt hai lớp của 3.3; helper đọc `(&Store, &ScopeResolver)` từ `OpenWorkState` `:54`; `mod wire` + `try_state`.
- `src-tauri/src/commands/chapter.rs:36` — `source_lang` **đã** đi lên frontend; `:119`/`:278` chỗ điền từ `open.meta.source_lang` — **đây là đường mở Chương để hâm `Jieba`**.
- `src-tauri/src/core/segment/split.rs:219` — `source_lang == LANG_CHINESE` ⇒ nhánh Trung, **mọi giá trị khác** ⇒ nhánh Anh. Cùng phép chọn, đừng đúc phép thứ hai.
- `src-tauri/tests/glossary_boundary.rs` — `GLOSSARY_ONLY_SURFACE` `:98` (ba tên, **không nới**) · `QUICK_ADD_SURFACE` `:~112` (**khuôn danh sách được phép** để chép) · `RS_FLOOR` `:55`.
- `src-tauri/tests/matching_boundary.rs` — `the_jieba_dictionary_is_constructed_at_exactly_one_place` · `…_lazily_initialised_once` — hai cổng giữ cho chi phí 243 ms không nhân lên.
- `src-tauri/src/core/i18n/mod.rs` — `macro_rules! message_keys!` `:62-91`, khuôn khai `:143-151`; `IpcError::new` `:369` là chỗ **DUY NHẤT** `message_key` gặp `params`.
- `src-tauri/src/lib.rs` — `generate_handler!` `:302-364`. ⚠️ **Không test nào bắt lỗi quên đăng ký.**
- `src-tauri/tests/ipc_contract.rs:228` `every_message_key_exists_in_vi_json` · `:321` khớp `params` ↔ placeholder hai chiều.
- Sổ nợ chủ **Story 3.4**, đọc trước khi viết dòng đầu: `:243` (`ScopeResolver` chưa cache — **đo trước khi cache**) · `:413` (Jieba lạnh) · `:422` (`-er`/`-est`) · `:424` (`find_terms` O(thuật ngữ × văn bản), **chưa đo**) · `:431` (tokenize hai lần) · `:5330-5345` (chính sách chuẩn hoá `source_term`).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/glossary/store.rs` -- hàm phơi ra **thứ tư** `marks_for_source_text(resolver, global, work, text, lang)`: tra hai tầng qua `apply_override` (**không** lọc `is_confirmed`), gọi `find_terms`, quy đổi span **byte → điểm mã**, phân xử chồng nhau (**dài nhất thắng, hoà thì trái nhất**), trả `Vec<GlossaryMark>` mang `(start, end, tier, is_confirmed, translation)` -- ba tên cũ ở lại bị cấm; đây là đường Ice đã ký hai lần thay vì nới cổng.
- [x] `src-tauri/src/core/glossary/store.rs` -- hàm hâm nóng `Jieba` gọi được từ ngoài, doc-comment dẫn thẳng số đo `deferred-work.md:413` -- 179–329 ms rơi vào **lần gọi đầu tiên**, tức có thể rơi đúng phím đầu người dùng gõ.
- [x] `src-tauri/src/commands/glossary.rs` -- hàm thuần `glossary_marks_for_chapter` + vỏ `mod wire` dùng `try_state`; hâm `Jieba` móc vào **đường mở Chương**, **không** trong thân hàm khớp -- khuôn hai lớp là thứ cho `tests/**` gọi được không cần webview.
- [x] `src-tauri/src/lib.rs` -- đăng ký command vào `generate_handler!` -- không test nào bắt lỗi quên bước này, nên nó là bước dễ mất nhất trong cả story.
- [x] `src-tauri/src/core/i18n/mod.rs` + `src/i18n/vi.json` -- **quyết định thực thi: KHÔNG thêm khoá mới.** Bốn khoá đã có (`store.open_failed` · `store.read_failed` · `store.write_failed` qua `GlossaryError::Store`, `glossary.scope_error` qua `GlossaryError::Scope`) diễn đạt trọn mọi nhánh lỗi mà `marks_for_source_text`/`glossary_marks_for_chapter` có thể trả -- không nhánh lỗi mới nào cần một khoá mới. Thêm một khoá không có nhánh nào đi qua là đúng thứ Story 1.7 §CN #3 cấm ("không khoá nào cho một tính năng chưa tồn tại"). Lý do đầy đủ ghi tại doc-comment của `glossary_marks_for_chapter` (`commands/glossary.rs`).
- [x] `src-tauri/tests/glossary_boundary.rs` -- thêm hàm thứ tư vào danh sách **được phép** theo khuôn `QUICK_ADD_SURFACE`; giữ `GLOSSARY_ONLY_SURFACE` ở ba tên; một **đối chứng dương** rằng cổng vẫn đỏ nếu ai gọi `load_tier` từ `commands/**` -- một cổng chưa bao giờ đỏ là một cổng chưa ai biết nó có chạy không.
- [x] `src-tauri/tests/glossary_marks_contract.rs` -- mới: mọi hàng của I/O Matrix, tên hàm là một **CÂU** khẳng định; kèm ca giới hạn **có tên** cho `happiest` (Ice ký 2026-08-21) -- ghi giới hạn ra thay vì để người sau tưởng nó đã được xét.
- [x] `scripts/check-i18n.mjs` · `glossary_boundary.rs:55` -- đo lại bằng chính cổng: số THẬT KHÔNG đổi (51 tệp `.rs`/17 `.vue` cho `check-i18n.mjs`; 50 tệp `.rs` cho `glossary_boundary.rs`) -- story không thêm tệp `.rs`/`.vue` sản phẩm nào, và tệp `tests/**` mới miễn trừ trọn khỏi cả hai quần thể. Sàn giữ nguyên, kèm ghi chú đo lại + ngày tại chỗ. 🔵 Sửa 2026-08-21: bản đầu viết *"hai tệp `tests/**` mới"* — tệp đo tạm `zzz_scratch_bench_marks.rs` đã xoá sau khi lấy xong bảng số, nên còn **một**; hai chú thích trong mã mang cùng mệnh đề sai đó đã sửa tại chỗ.
- [x] `_bmad-output/implementation-artifacts/sprint-status.yaml` -- đổi khoá `3-4-…-trong-panel-source` → `3-4-khop-thuat-ngu-theo-ngon-ngu-qua-matcher-dung-chung`, kèm dòng 🔵 + ngày + lý do theo khuôn "Nhật ký sprint-status" (`:106`, `:138`) -- FR50 sửa **2026-08-18** (`epics.md:176`, `prd.md:548`) và khoá là chỗ **cuối cùng** còn mang tên đã hết đúng; tên mới cũng phải nói đúng phạm vi đã thu hẹp.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- đóng `:422` và `:5330-5345` bằng `→ KHÔNG LÀM 2026-08-21` kèm **điều gì đã đổi**; nối 🟡 vào `:243` · `:424` · `:431` với **số đo thật** lấy ở story này; giữ nguyên mục nửa giao diện vừa mở -- `check:debt-owner` đỏ với mục mồ côi, và không bao giờ xoá một mục đã đóng. ⚠️ Số đo tại `:424` **vượt trần NFR2** ở quy mô Chương lớn nhất có thật (214 ms so với 50 ms, Glossary 5.000 mục) — cửa ASK-FIRST của story kích hoạt; xem §Verification và báo cáo hoàn tất cho quyết định cần Ice.

**Acceptance Criteria:**
- Given một `grep` trên `src-tauri/src/**`, when tìm `insert_manual_entry`/`confirm_translation`/`load_tier`, then không tệp nào ngoài `core/glossary/**` gõ ba tên đó.
- Given diff của story, when đọc, then **không dòng nào** cài lại phép khớp ngoài `core/matching/**` (AD-17), và `core/dict/**` vẫn có **0** lời gọi tới nó.
- Given `deferred-work.md:413`, when đọc mã sản phẩm, then lượt hâm `Jieba` nằm trên đường **mở Chương**, không trên đường khớp, và doc-comment dẫn thẳng số đo.
- Given `find_terms` chạy trên một Glossary cỡ thật, when đo, then con số **được ghi vào** `deferred-work.md:424` — mục đó nói *"chưa đo — chưa có người tiêu thụ nào để đo trên đó"*, và story này **là** người tiêu thụ đó.
- Given `.githooks/pre-push`, when chạy, then mười một cổng + vitest + build + `cargo test --locked` đều xanh; `check:gates` không đổi vì story này **không thêm cổng**.
- Given lượt CI sau khi push, when đọc, then cả nửa macOS lẫn nửa Windows xanh — `pre-push` chỉ chạy trên macOS của Ice.

## Spec Change Log

### 2026-08-21 — thực thi

**Quyết định thực thi không tường minh trong spec, ghi lại để lượt sau đọc được lý do:**

- **`glossary_marks_for_chapter` nhận `text: &str` + `source_lang: &str` làm THAM SỐ, không tự
  đọc `chapter` từ đĩa qua `chapter_id`.** Cùng khuôn `glossary_lookup_term` (nhận `source_term`
  làm tham số): frontend đã có `source_text`/`source_lang` từ `read_open_chapter` trước khi gọi
  lệnh này. Thiết kế này là điều kiện CẤU TRÚC để thoả đúng hàng I/O Matrix *"Chưa mở Tác phẩm
  ⇒ Chỉ khớp tầng Global, không lỗi"* — nếu hàm tự đọc Chương từ `OpenWork.chapter_id`, nó buộc
  phải đòi một Tác phẩm đang mở (không có Chương nào để đọc khi chưa có Tác phẩm) và hàng đó
  không thoả được.
- **KHÔNG thêm `message_key` mới ở `core::i18n`.** Bốn khoá đã có
  (`store.open_failed`/`read_failed`/`write_failed` qua `GlossaryError::Store`,
  `glossary.scope_error` qua `GlossaryError::Scope`) diễn đạt trọn mọi nhánh lỗi mà
  `marks_for_source_text` có thể trả — hàm đó chỉ gọi `load_tier` và
  `ScopeResolver::apply_override`. Thêm một khoá không có nhánh nào đi qua là đúng thứ Story
  1.7 §Completion Notes #3 cấm.
- **Phân xử chồng nhau: thuật toán THAM LAM, sắp `(độ dài giảm dần, vị trí bắt đầu tăng dần)`
  rồi chọn tuần tự, bỏ qua ứng viên chồng lấn một lượt đã chọn.** Spec chỉ nêu quy tắc ("dài
  nhất thắng, hoà thì trái nhất"), không nêu thuật toán; đây là cách viết tất định đơn giản
  nhất thoả đúng quy tắc đó, và độ phức tạp O(n²) trên SỐ LƯỢT KHỚP (không phải trên độ dài văn
  bản) được chấp nhận vì số lượt khớp trong một Chương thực tế nhỏ hơn nhiều so với độ dài văn
  bản.
- **Hâm `Jieba` gọi từ CẢ HAI hàm mở Chương** (`read_open_chapter` VÀ `open_adjacent_chapter`),
  không chỉ một. Cả hai đều là đường sản phẩm đưa một `source_lang` mới lên webview; gọi lặp
  không tốn gì (`LazyLock` chỉ dựng một lần, ~1 µs từ lần gọi thứ hai) nên hâm ở cả hai chỗ an
  toàn hơn hâm ở một chỗ rồi phải nhớ thêm chỗ thứ hai sau này.
- **`GlossaryMark`/`GlossaryMarkWire` không mang `source_term` hay `id`.** Bốn trường
  `(start, end, tier, is_confirmed, translation)` đúng như spec liệt kê là đủ để vẽ dấu; thêm
  `id` sẽ mời một lượt "sửa nhanh từ dấu" mà spec không đòi và nửa giao diện (3.4b) chưa định
  hình.
- **`QUICK_ADD_SURFACE` (`glossary_boundary.rs`) MỞ RỘNG thành 4 phần tử, không đổi tên.** Tên
  hằng nay hơi rộng hơn nghĩa gốc ("quick add") vì nó phục vụ cả bề mặt đánh dấu 3.4, nhưng đổi
  tên sẽ chạm hai test đã có mà không đổi mệnh đề chúng canh — giữ tên, ghi 🔵 + ngày giải thích
  tại chỗ, đúng luật "sửa tại chỗ" hơn là "đổi tên cho đẹp".
- 🔴 **PHÁT HIỆN ĐO ĐẠC, KHÔNG PHẢI QUYẾT ĐỊNH ĐÃ CHỐT — đọc trước khi coi story này "xong":**
  `marks_for_source_text` đo trên Glossary 5.000 mục, Chương 48.640 ký tự (**Chương lớn nhất
  có thật**, `commands/segment.rs:1111`) cho **214 ms trung vị** — **~4,3× trần NFR2** (50 ms).
  Số đo đầy đủ + toolchain ở `deferred-work.md:424`. Đây đúng là điều kiện ASK-FIRST mà chính
  §Boundaries của spec này đặt tên trước ("Nếu phép đo `find_terms` … cho thấy cần chỉ mục
  ngược hoặc cache — đo trước, chốt sau"): story dừng lại ở việc ĐO, **không** tự thêm cache
  hay chỉ mục ngược. Quyết định thuộc về Ice, và phụ thuộc một dữ kiện chưa biết — tần suất gọi
  lại của nửa giao diện (3.4b): mở Chương một lần hay mỗi lượt gõ.

## Design Notes

**Ba đơn vị đo, và chỗ quy đổi là một quyết định:**

```
Rust / Matcher  -> BYTE     (find_terms trả Range<usize> byte)
Dây  / lưới     -> ĐIỂM MÃ  (sourcePieceStartsOf · sourceCutOffsetOf · regroup::split_at)
DOM  / Range    -> UTF-16   (mọi offset của Range)
```

⇒ Quy đổi byte → điểm mã làm **trong Rust**. Để nó cho frontend là mời một lượt lệch im lặng ở đúng chỗ chuỗi JS lại là một đơn vị **thứ ba**.

⚠️ **`find_terms` trả span CHỒNG NHAU được** — `AA` trong `AAA` là hai lượt xuất hiện thật, và hai thuật ngữ khác nhau phủ lên nhau cũng vậy. Một kênh **đánh dấu** thì không phân thân được: phải phân xử **ở đây**, không đẩy xuống cho nửa giao diện tự nghĩ ra một luật thứ hai. Chọn **dài nhất thắng, hoà thì trái nhất** — tất định, và nó ưu tiên thuật ngữ **cụ thể hơn**.

🔵 **Chính sách chuẩn hoá `source_term` (`deferred-work.md:5330`) đóng bằng một phép đo, không bằng một lượt sửa lược đồ:** `find_terms` nhánh `En` đã chuẩn hoá **cả hai vế** (hạ chữ thường rồi Porter2) nên `Fire`/`fire` gặp nhau **ở đường khớp**; nhánh `Zh` khớp chính xác nên không có gì để chuẩn hoá. ⇒ Bảng **không** chuẩn hoá — đúng luật *"hạ chữ thường là THÊM một khoá, không THAY khoá gốc"*, và nó giữ được 1.635 đầu mục tiếng Anh có chữ hoa mang nghĩa. Một `UNIQUE` chuẩn hoá thì **không lùi được** sau khi dữ liệu người dùng đã nằm trên đĩa.

## Verification

**Commands:**
- `cd src-tauri && cargo test --locked` -- expected: xanh, gồm `glossary_marks_contract`, `glossary_boundary`, `matching_boundary`, `ipc_contract`.
- `npm run build && npm run test` -- expected: xanh; thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch, không ở một assert.
- `npm run check:i18n` · `check:debt-owner` -- expected: xanh sau khi nâng sàn.

**Manual checks (if no CLI):**
- **Đo `find_terms` trên một Glossary cỡ thật và một Chương thật**, ghi số + ngày + toolchain vào `deferred-work.md:424`. Một con số đo trên đầu vào tự bịa là một con số không dùng được.
- Đọc lượt CI trên GitHub: `pre-push` xanh trên macOS không nói gì về nửa Windows.
  → ✅ **ĐỌC RỒI 2026-08-21 — run `32453862410` trên `56859a9`: `check (macos-26)` success ·
  `check (windows-2025)` success.** AC cuối của story đóng bằng một lượt chạy thật, không bằng
  suy luận từ `pre-push` (thứ chạy 128 s trên macOS của Ice và theo đúng doc-comment của chính
  nó *"KHÔNG nói được gì về nửa Windows"*). ⚠️ **Vế KHÔNG đóng, ghi ra thay vì để tưởng nhầm:**
  job `e2e (macos-26)` ở lượt này là **`skipped`** — nó chỉ chạy theo nhịp đêm và khi bấm tay,
  đúng như `.githooks/pre-push:30-36` đã khai. Nên không một mệnh đề "webview thật" nào của kho
  được lượt CI này nghiệm thu; story 3.4 không thêm mệnh đề nào loại đó, nhưng câu này ở lại để
  lượt sau không đọc "CI xanh" thành "e2e đã chạy".

## Suggested Review Order

**Bất biến trung tâm — hàm phơi ra THỨ TƯ, và ba tên cấm vẫn bị cấm**

- Điểm vào của cả story: tra hai tầng, khớp, phân xử, quy đổi — một chỗ.
  [`store.rs:771`](../../src-tauri/src/core/glossary/store.rs#L771)

- Danh sách CHO PHÉP lên bốn tên; `GLOSSARY_ONLY_SURFACE` vẫn đúng ba.
  [`glossary_boundary.rs:125`](../../src-tauri/tests/glossary_boundary.rs#L125)

- Bốn trường đủ để VẼ một dấu, cố ý không mang `id`/`source_term`.
  [`entry.rs:236`](../../src-tauri/src/core/glossary/entry.rs#L236)

**Ba đơn vị đo — chỗ dễ cài lệch im lặng nhất**

- Quy đổi byte → điểm mã làm trong Rust; DOM lại đếm UTF-16, đơn vị thứ ba.
  [`store.rs:694`](../../src-tauri/src/core/glossary/store.rs#L694)

- Nhánh `Err` của `binary_search` là lưới, không phải đường thật — lý do ghi tại chỗ.
  [`store.rs:704`](../../src-tauri/src/core/glossary/store.rs#L704)

**Chồng nhau — luật RIÊNG của kênh đánh dấu, không phải của phép khớp**

- Dài nhất thắng, hoà thì trái nhất; `find_terms` KHÔNG bị sửa vì TM cũng dùng nó.
  [`store.rs:722`](../../src-tauri/src/core/glossary/store.rs#L722)

**NFR2 — chi phí 179–329 ms được DỜI, không bị xoá**

- `LazyLock::force` sống cạnh `JIEBA`; đưa sang `glossary` sẽ phải phơi nó ra.
  [`matching/mod.rs:163`](../../src-tauri/src/core/matching/mod.rs#L163)

- Chỉ hâm cho Chương tiếng Trung — hâm cho tiếng Anh là trả 243 ms không ai hưởng.
  [`store.rs:679`](../../src-tauri/src/core/glossary/store.rs#L679)

- Đường mở Chương thứ nhất; lời gọi ở đây là thứ cổng mới canh.
  [`chapter.rs:117`](../../src-tauri/src/commands/chapter.rs#L117)

- Phép chọn ngôn ngữ nay viết ra ĐÚNG MỘT lần trong `glossary`.
  [`store.rs:657`](../../src-tauri/src/core/glossary/store.rs#L657)

**Bề mặt IPC — khuôn hai lớp**

- Hàm thuần: `tests/**` gọi được không cần webview.
  [`glossary.rs:258`](../../src-tauri/src/commands/glossary.rs#L258)

- `try_state` chứ không `state()`; `None` chỉ là nhánh chưa `manage`, không phải đường chính.
  [`glossary.rs:385`](../../src-tauri/src/commands/glossary.rs#L385)

- Bước dễ mất nhất cả story — không test nào bắt lỗi quên đăng ký.
  [`lib.rs:375`](../../src-tauri/src/lib.rs#L375)

**Cổng mới — và bằng chứng nó ĐỎ ĐƯỢC**

- Quét nguồn, không đo thời gian: một ngưỡng ms sẽ chập chờn trên runner đang tải.
  [`matching_boundary.rs:585`](../../src-tauri/tests/matching_boundary.rs#L585)

- Đối chứng dương; đã xoá thật lời gọi rồi chạy lại — cổng đỏ và gọi đích danh hàm.
  [`matching_boundary.rs:618`](../../src-tauri/tests/matching_boundary.rs#L618)

**Test — bốn ca đáng đọc nhất**

- AD-18 đi qua CHÍNH vỏ IPC, không qua resolver lắp tay.
  [`glossary_marks_contract.rs:507`](../../src-tauri/tests/glossary_marks_contract.rs#L507)

- Khẳng định cả `translation` lẫn `tier`: ánh xạ `terms`↔`payload` xáo trộn sẽ đỏ.
  [`glossary_marks_contract.rs:297`](../../src-tauri/tests/glossary_marks_contract.rs#L297)

- Kho tầng Tác phẩm đóng giữa chừng — nhánh lỗi RIÊNG, không phải nhánh Global.
  [`glossary_marks_contract.rs:464`](../../src-tauri/tests/glossary_marks_contract.rs#L464)

- Giới hạn Porter2 ghi thành ca CÓ TÊN, không giấu trong một ca AC.
  [`glossary_marks_contract.rs:144`](../../src-tauri/tests/glossary_marks_contract.rs#L144)
