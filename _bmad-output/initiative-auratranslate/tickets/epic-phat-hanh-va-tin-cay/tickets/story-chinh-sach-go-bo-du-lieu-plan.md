---
title: 'Story 10.8 — Chính sách gỡ bỏ dữ liệu'
type: 'feature'
ticket: '8'
created: '2026-10-21'
status: 'built'
baseline_revision: '2e488d7d515525a4efe5c367dc2af0f27c93568f'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR112 hứa gỡ một nguồn bằng xoá một `.db` khỏi `dict-manifest.toml`, không đổi mã. Ứng dụng đã đáp ứng (quét `$RESOURCE/dict/*.db`, Attribution dựng từ `dict_source` của tệp có mặt), nhưng ba cổng chép cứng danh sách lớp nên gỡ một lớp làm CI đỏ cho tới khi sửa mã; `inspect-release` không bắt một `.db` thừa nên "khỏi bản đóng gói" không được canh; chưa có quy trình viết ra.

**Approach:** Cho các cổng lấy danh sách lớp từ dữ liệu thay vì hằng trong mã, thêm vế "không `.db` nào ngoài manifest" vào `inspect-release`, viết quy trình gỡ trong repo, rồi diễn tập gỡ thật một lớp gỡ rời trên dữ liệu thật.

**Decisions (Ice, 2026-10-21):**
- Q1 = A. Ý định gỡ ghi trong manifest: khối của lớp chuyển thành `[[removed]]` với `name`, `reason`, `date`. Cổng đòi `detachable ∪ removed = DETACHABLE_ALL` và hai tập rời nhau. Xoá khối mà không ghi `[[removed]]` vẫn đỏ.
- Q2 = L1. Giấy phép riêng của lớp đã gỡ vẫn được đóng gói (`assets/licenses/*.txt` không đổi). Quy trình nói rõ điều này. `NOTICE.txt` sửa tay.
- Q3 = R1. Diễn tập gỡ `tran-van-chanh` trong một worktree bỏ đi, trên dữ liệu thật. Master giữ bốn lớp.

## Boundaries & Constraints

**Always:** sau story, gỡ một lớp chỉ đụng tệp dữ liệu/tài liệu (`dict-manifest.toml`, `.md`, `NOTICE.txt`), không `.rs`/`.ts`/`.vue`/`.mjs`/`.json`; một lớp RƠI MẤT do vô ý vẫn phải làm một cổng đỏ; mọi cổng giữ sàn chống cây rỗng hiện có.

**Never:** sửa đường nạp lớp lúc chạy (`DictLayers::open`) hay Attribution — chúng đã đúng; xoá tệp giấy phép khỏi `tools/dict-build/assets/licenses/` (`licenses.rs` `include_str!`); dựng lớp HVTĐTD (không có nguồn, Ice chốt 2026-08-08); commit một `.db`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Gỡ có chủ ý | khối `[[detachable]]` chuyển thành `[[removed]]`, `.db` không còn trong `resources/dict/` | `check:dict-manifest`, `check:dict-build`, `config_invariants` xanh không sửa mã | — |
| Rơi vô ý | xoá khối `[[detachable]]`, không có `[[removed]]` | `check:dict-manifest` và Kiểm D đỏ | thông báo nêu đúng tên lớp |
| Ghi hai nơi | cùng `name` ở cả `[[detachable]]` lẫn `[[removed]]` | đỏ | nêu tên trùng |
| Bia mộ thiếu trường | `[[removed]]` thiếu `reason` hoặc `date`, hoặc `name` lạ ngoài `DETACHABLE_ALL` | đỏ | nêu mục và trường |
| `.db` thừa trong bundle | `resources/dict/` còn tệp của lớp đã gỡ | `inspect-release` đỏ, nêu tên tệp | — |
| Dữ liệu người dùng trỏ nguồn đã gỡ | mục đã ghim / nguồn đã tắt mang mã lớp vừa gỡ | danh sách ghim vẫn hiện, tra cứu không lỗi | đo ở diễn tập, lệch thì ghi nợ |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/dict/layer.rs:582` `DictLayers::open` — quét thư mục, không danh sách tên; không đổi.
- `src-tauri/src/commands/dict.rs:336` `list_dict_sources` → `AttributionOverlay.vue:253` — Attribution từ `dict_source`; không đổi.
- `scripts/check-dict-manifest.mjs:132` `EXPECTED_DETACHABLE_NAMES` — danh sách cứng thứ nhất.
- `scripts/check-dict-build.mjs:266-343` Kiểm D — so `DETACHABLE_ALL` (`tools/dict-build/src/sources_meta.rs:215`, test `:293`) với `[[detachable]]`, đòi bằng nhau; giữ sàn ở cùng khối.
- `src-tauri/tests/config_invariants.rs:654-664` — chép cứng bốn tên tệp từ manifest.
- `scripts/lib/dict-fetch.mjs:138` `inspectInstalledTree` — chỉ đếm tệp manifest, không bắt tệp thừa; `fetch-dicts.mjs` không dọn `resources/dict/`. `release.yml` checkout sạch nên CI không dính, máy cục bộ thì dính.
- `scripts/lib/dict-fetch.mjs` `inspectInstalledLicenses` (`MIN_BUNDLED_LICENSES = 9`), `tauri.conf.json:40` glob `assets/licenses/*.txt` — `thieu-chuu.txt`·`vietphrase.txt`·`tran-van-chanh.txt` là giấy phép riêng của lớp; theo L1 chúng vẫn được đóng gói, không đổi.
- `NOTICE.txt:12-13,29,35-38` — nhắc `dict-tran-van-chanh.db`; tài liệu, không cổng nào đọc.
- `src-tauri/tests/dict_sources.rs:2045,2094,2163,3032` — đã có: lớp dạng Hán Việt đứng cạnh lớp nền, xoá từng lớp fixture thì bộ tra cứu vẫn xanh và rơi về nhãn tiếng Anh (`pos_lang='en'`). Đây là phủ của AC HVTĐTD; không có lớp HVTĐTD thật để gỡ.
- `src-tauri/tests/dict_sources.rs:2274,3786,4764` — ca `#[ignore]` đọc dữ liệu thật qua `AURA_DICT_BENCH_DIR`; dùng cho diễn tập.
- `src-tauri/resources/dict/README.md:13` — còn ghi ba tệp, sai.
- `deferred-work.md:858` — rủi ro pháp lý `dict-tran-van-chanh.db`, `Chủ: Ice`.

## Tasks & Acceptance

**Execution:**
- [ ] `scripts/check-dict-manifest.mjs` -- bỏ `EXPECTED_DETACHABLE_NAMES`; kiểm dạng `[[removed]]` (mảng bảng, `name`/`reason`/`date` không rỗng, không `url`/`sha256`), tên rời nhau giữa hai mảng, và `detachable ∪ removed` khớp `DETACHABLE_ALL` đọc từ `sources_meta.rs` theo cách Kiểm D đang đọc -- danh sách cứng là thứ buộc sửa mã.
- [ ] `scripts/check-dict-build.mjs` -- Kiểm D so `DETACHABLE_ALL` với `detachable ∪ removed`, giữ sàn -- builder vẫn dựng được lớp đã gỡ.
- [ ] `scripts/lib/dict-fetch.mjs` -- `manifestEntries` bỏ qua `[[removed]]` (không tải, không kiểm) và `inspectInstalledTree` đỏ khi dưới một thư mục `dict` có `.db` không thuộc manifest; thêm ca vào test vitest hiện có của tệp này.
- [ ] `src-tauri/tests/config_invariants.rs` -- thay bốn tên chép cứng bằng ràng buộc dẫn từ manifest: base là `dict-core.db`, mọi tên khớp glob `*.db`, không trùng, không tên nào thuộc `[[removed]]`.
- [ ] `dict-manifest.toml` -- thêm chú thích mô tả `[[removed]]`; hôm nay chưa có mục nào.
- [ ] `DATA-REMOVAL.md` (gốc repo, tiếng Việt) -- quy trình gồm: chuyển khối sang `[[removed]]`, xoá `.db` cục bộ, sửa `NOTICE.txt`, các lệnh kiểm, cách xác nhận Attribution, việc giấy phép riêng vẫn được đóng gói (L1), phạm vi HVTĐTD; trỏ FR112, FR36, AD-25.
- [ ] `src-tauri/tests/config_invariants.rs` -- ca giữ `DATA-REMOVAL.md` nêu đúng tệp và lệnh cổng mà quy trình dựa vào (cùng kiểu ca `install_guide_keeps_…`).
- [ ] `src-tauri/resources/dict/README.md` -- sửa "ba tệp".
- [ ] Diễn tập (R1, worktree bỏ đi, `tran-van-chanh`) -- chạy quy trình trên dữ liệu thật: cổng, ca `#[ignore]` thật với thư mục thiếu lớp đó, `tauri build --config tauri.release.conf.json --bundles app` + `inspect-release`, mở Attribution; ghi kết quả vào Implementation Notes.

**Acceptance Criteria:**
- Given quy trình trong `DATA-REMOVAL.md`, when gỡ một lớp gỡ rời, then `git diff` chỉ chạm tệp dữ liệu/tài liệu và mọi cổng liên quan xanh.
- Given thư mục từ điển thật thiếu lớp vừa gỡ, when chạy các ca tra cứu dữ liệu thật, then xanh và danh sách nguồn không còn nguồn của lớp đó.
- Given bản `.app` dựng sau khi gỡ, when mở Attribution, then ghi công của lớp đó không còn; when chạy `inspect-release`, then xanh, và đỏ nếu đưa lại tệp `.db` cũ vào `dict/`.
- Given HVTĐTD bị gỡ, when tra Hán Việt, then rơi về nhãn tiếng Anh — phủ bởi `dict_sources.rs:2045/2163` trên fixture, ghi rõ trong quy trình.
- Mỗi cổng mới/sửa có đối chứng gỡ thật (AGENTS.md).

## Implementation Notes

- Danh sách lớp lấy từ `DETACHABLE_ALL` qua `scripts/lib/detachable-all.mjs`, dùng chung cho `check-dict-manifest` và Kiểm D; Kiểm F bỏ qua lớp ở `[[removed]]` (builder vẫn dựng được lớp đó).
- `inspectInstalledTree` đỏ khi có `.db` dưới thư mục `dict` ngoài manifest; `manifestEntries` vốn chỉ đọc `[base]` + `[[detachable]]` nên bỏ qua `[[removed]]`.
- Diễn tập R1 (worktree bỏ đi, gỡ `tran-van-chanh`): `check:dict-manifest`, `check:dict`, `config_invariants` (49 ca) xanh; ba ca `#[ignore]` của `dict_sources` xanh với thư mục thiếu lớp đó; `tauri build --config tauri.release.conf.json --bundles app` ra `.app` chỉ 3 `.db`, `inspect-release` xanh, và đỏ đúng tên tệp khi đưa lại `dict-tran-van-chanh.db`.
- Đối chứng giữ trong repo bằng `tests/frontend/detachableAll.test.ts` (đọc `sources_meta.rs` thật; rơi mất, ghi hai nơi, bia mộ thiếu `reason`/`date`/mang `url`/tên lạ); gỡ thật nhánh `missing` thì đúng ca "neither array" đỏ.
- Dữ liệu người dùng trỏ lớp đã gỡ: `list_pinned_entries` chỉ đọc `Store`, không nối với lớp nạp, nên ghim vẫn hiện (ca mới `a_pin_of_a_source_whose_layer_file_was_removed_is_still_listed`); mã nguồn đã tắt không có tệp thì bỏ qua im lặng (ca sẵn `a_disabled_code_with_no_file_behind_it_is_ignored_in_silence`). Không lệch.
- Chưa làm: mở Attribution bằng tay trên `.app` diễn tập (phủ gián tiếp bằng ca dữ liệu thật).

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 2 · low 5 · false 2 · maybe-false 0.
- medium · patch — Attribution trên `.app` diễn tập chưa mở, không có mục nợ: đúng, việc chỉ người làm được phải thành mục `Chủ: Epic 10`; thêm mục nợ.
- medium · patch — ba ca `#[ignore]` là bench, không khẳng định danh sách nguồn thiếu lớp đã gỡ; thêm ca `#[ignore]` dữ liệu thật khẳng định điều đó và chạy nó.
- low · patch — `manifest_removed_names` khớp mọi khoá bắt đầu bằng `name`; sửa thành `name` rồi `=`. Vế "không tên nào thuộc `[[removed]]`" giữ vì plan đòi, nó đỏ khi một tên ở cả hai mảng.
- false — Kiểm D không bắt tên ghi hai nơi: `check:dict-manifest` bắt và chạy trong `pre-push` (`.githooks/pre-push:53`) lẫn CI.
- low · reject — hai bộ quét dòng trong `check-dict-build.mjs` không nhận `[[removed]] # ghi chú`: cả hai đều đỏ ầm (Kiểm D "thiếu", Kiểm F không thấy `source_version`), mẫu `[[detachable]]` cũ cũng vậy; sửa đòi đổi parser.
- low · patch — chú thích đầu `check-dict-manifest.mjs` còn "ĐÚNG N mục", `resources/dict/README.md` còn câu `dict-v1` ba tệp; sửa tại chỗ.
- medium→low · patch — `NOTICE.txt` đi vào bản cài nhưng trỏ `DATA-REMOVAL.md` không đóng gói và thay câu "deleting that one file" vẫn đúng lúc chạy; trả về nguyên bản.
- low · patch — comment mới/sửa trong `check-dict-build.mjs` và `check-dict-manifest.mjs` viết tiếng Việt, trái AGENTS.md; đổi sang tiếng Anh hoặc xoá câu chỉ thuật lại mã.
- false — regex tệp thừa của `inspectInstalledTree`: lớp phủ release chỉ đặt `resources/dict/*.db` phẳng vào `dict/`, không có `.db` lồng sâu hay thư mục `dict` khác trong bản cài; nếu có, đỏ ầm là đúng.

## Design Notes

Danh sách cứng trong cổng đang làm hai việc: bắt một lớp rơi vô ý, và buộc sửa mã khi gỡ có chủ ý. FR112 chỉ cấm việc thứ hai. `[[removed]]` dời "ý định gỡ" từ mã sang dữ liệu mà vẫn giữ việc thứ nhất:

```toml
[[removed]]
name   = "tran-van-chanh"
reason = "khiếu nại bản quyền …"
date   = "2026-10-21"
```

## Verification

**Commands:**
- `npm run check:dict-manifest && npm run check:dict-build` -- xanh trên cây thật; đỏ đúng tên khi gỡ lớp sai cách.
- `cargo test --test config_invariants` -- xanh.
- `npx vitest run tests/frontend/fetchDicts.test.ts` (hoặc tệp test `dict-fetch` tương ứng) -- xanh.

**Manual checks (if no CLI):**
- Attribution trên `.app` diễn tập không còn lớp đã gỡ (Ice hoặc ảnh chụp kèm).

